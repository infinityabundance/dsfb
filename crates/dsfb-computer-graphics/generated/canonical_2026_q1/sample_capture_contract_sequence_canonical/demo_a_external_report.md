# Demo A External Report

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

Point-vs-region disclosure is explicit per capture via the `point_vs_region` line below.

ROI is defined as pixels where baseline error exceeds 15% of local contrast. The mask is computed once from the baseline and held fixed across all methods. DSFB does not influence ROI selection.

## Capture `frame_0001`

- ROI source: `fixed_alpha_local_contrast_0p15`
- ROI pixels: `14159`
- ROI coverage: `38.41%`
- ROI baseline method: `fixed_alpha`
- reference_source: `reference_color`
- ground_truth_available: `true`
- metric_source: `real_reference`
- point_vs_region: `region_like`
- realism_stress_note: `realism_stress_case`
- larger_roi_note: `larger_roi_case`

| Method | full-frame MAE | ROI MAE | non-ROI MAE | max error | temporal error accumulation | intervention rate |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Fixed alpha baseline | 0.11153 | 0.28734 | 0.00190 | 0.60685 | 0.11153 | 0.00000 |
| Strong heuristic clamp | 0.00413 | 0.00844 | 0.00145 | 0.30008 | 0.00413 | 0.33339 |
| DSFB host minimum | 0.02148 | 0.05302 | 0.00182 | 0.30050 | 0.02148 | 0.21345 |
| DSFB + strong heuristic | 0.00333 | 0.00635 | 0.00145 | 0.30008 | 0.00333 | 0.33373 |

## Capture `frame_0002`

- ROI source: `fixed_alpha_local_contrast_0p15`
- ROI pixels: `27619`
- ROI coverage: `74.92%`
- ROI baseline method: `fixed_alpha`
- reference_source: `reference_color`
- ground_truth_available: `true`
- metric_source: `real_reference`
- point_vs_region: `region_like`
- realism_stress_note: `realism_stress_case`
- larger_roi_note: `larger_roi_case`

| Method | full-frame MAE | ROI MAE | non-ROI MAE | max error | temporal error accumulation | intervention rate |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Fixed alpha baseline | 0.34025 | 0.45358 | 0.00170 | 0.60852 | 0.34025 | 0.00000 |
| Strong heuristic clamp | 0.00222 | 0.00265 | 0.00094 | 0.20031 | 0.00222 | 0.69737 |
| DSFB host minimum | 0.02962 | 0.03903 | 0.00150 | 0.29211 | 0.02962 | 0.64758 |
| DSFB + strong heuristic | 0.00175 | 0.00203 | 0.00094 | 0.19184 | 0.00175 | 0.69752 |

## Capture `frame_0003`

- ROI source: `fixed_alpha_local_contrast_0p15`
- ROI pixels: `9205`
- ROI coverage: `24.97%`
- ROI baseline method: `fixed_alpha`
- reference_source: `reference_color`
- ground_truth_available: `true`
- metric_source: `real_reference`
- point_vs_region: `region_like`
- realism_stress_note: `realism_stress_case`
- larger_roi_note: `larger_roi_case`

| Method | full-frame MAE | ROI MAE | non-ROI MAE | max error | temporal error accumulation | intervention rate |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Fixed alpha baseline | 0.07257 | 0.28668 | 0.00132 | 0.60450 | 0.07257 | 0.00000 |
| Strong heuristic clamp | 0.00296 | 0.00860 | 0.00108 | 0.31922 | 0.00296 | 0.21853 |
| DSFB host minimum | 0.01444 | 0.05388 | 0.00131 | 0.25549 | 0.01444 | 0.13224 |
| DSFB + strong heuristic | 0.00235 | 0.00615 | 0.00108 | 0.24858 | 0.00235 | 0.21878 |

## Capture `frame_0004`

- ROI source: `fixed_alpha_local_contrast_0p15`
- ROI pixels: `17073`
- ROI coverage: `46.31%`
- ROI baseline method: `fixed_alpha`
- reference_source: `reference_color`
- ground_truth_available: `true`
- metric_source: `real_reference`
- point_vs_region: `region_like`
- realism_stress_note: `realism_stress_case`
- larger_roi_note: `larger_roi_case`

| Method | full-frame MAE | ROI MAE | non-ROI MAE | max error | temporal error accumulation | intervention rate |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Fixed alpha baseline | 0.10623 | 0.22578 | 0.00309 | 0.59634 | 0.10623 | 0.00000 |
| Strong heuristic clamp | 0.00528 | 0.00852 | 0.00249 | 0.25255 | 0.00528 | 0.29559 |
| DSFB host minimum | 0.02105 | 0.04196 | 0.00302 | 0.23677 | 0.02105 | 0.20294 |
| DSFB + strong heuristic | 0.00440 | 0.00662 | 0.00248 | 0.21349 | 0.00440 | 0.29603 |

## Capture `frame_0005`

- ROI source: `fixed_alpha_local_contrast_0p15`
- ROI pixels: `25211`
- ROI coverage: `68.39%`
- ROI baseline method: `fixed_alpha`
- reference_source: `reference_color`
- ground_truth_available: `true`
- metric_source: `real_reference`
- point_vs_region: `region_like`
- realism_stress_note: `realism_stress_case`
- larger_roi_note: `larger_roi_case`

| Method | full-frame MAE | ROI MAE | non-ROI MAE | max error | temporal error accumulation | intervention rate |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Fixed alpha baseline | 0.27106 | 0.39490 | 0.00314 | 0.60392 | 0.27106 | 0.00000 |
| Strong heuristic clamp | 0.00398 | 0.00466 | 0.00252 | 0.30322 | 0.00398 | 0.56103 |
| DSFB host minimum | 0.02714 | 0.03824 | 0.00312 | 0.36979 | 0.02714 | 0.50715 |
| DSFB + strong heuristic | 0.00345 | 0.00388 | 0.00251 | 0.30322 | 0.00345 | 0.56137 |

## What Is Proven

- The same DSFB host-minimum supervisory layer runs on imported buffers and can be compared against fixed alpha, a strong heuristic baseline, and an explicit DSFB + strong heuristic hybrid.
- ROI and non-ROI behavior remain separated on imported data.
- The current real Unreal-native package also emits `figures/trust_temporal_trajectory.svg` and `figures/trust_temporal_trajectory.json` over the ordered capture sequence.

## What Is Not Proven

- The current bundle measures against `reference_color`, but that reference is a higher-resolution exported Unreal proxy rather than a path-traced ground truth.
- Even with a reference frame, this does not replace longer engine-side sequences.
- The current bundle generates `figures/trust_histogram.svg`, `figures/trust_vs_error.svg`, `figures/trust_conditioned_error_map.png`, and `figures/trust_temporal_trajectory.svg`; these are calibration artifacts over a short five-frame sequence, not a broad temporal generalization claim.

## Remaining Blockers

- Real engine capture sequences and longer temporal windows still need evaluation.
