# External Scaling Report

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

Native imported capture: `frame_0001` at 256x144.

| Label | Source | Resolution | Attempted | Measured GPU | total_ms | dispatch_ms | readback_ms | ms/MPixel | scaling ratio vs native | approx linear |
| --- | --- | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | --- |
| native_imported | native_imported | 256x144 | true | true | 19.4802 | 19.3927 | 0.0869 | 528.4336 | 1.0000 | true |
| scaled_1080p | scaled_subprocess_probe | 1920x1080 | true | true | 51.1168 | 48.7230 | 2.3929 | 24.6513 | 2.6240 | false |
| scaled_4k | scaled_subprocess_probe | 3840x2160 | true | true | 141.7441 | 133.4838 | 8.2579 | 17.0891 | 7.2763 | false |

Cost appears approximately linear with resolution: `false`.

## Coverage

- realism_stress_case: `true`
- larger_roi_case: `true`
- mixed_regime_case: `false`
- coverage_status: `partial`
- missing coverage labels: mixed_regime_case

## What Is Not Proven

- This scaling report does not replace full engine-side profiling on real exported captures.
- When a row is marked unavailable, the corresponding scaling point was attempted but not measured in the current environment.

## Remaining Blockers

- Imported-buffer scaling does not replace full in-engine profiling on the final evaluator hardware and renderer integration point.
