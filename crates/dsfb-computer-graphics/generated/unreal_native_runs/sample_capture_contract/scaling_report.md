# External Scaling Report

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

Native imported capture: `frame_0001` at 256x144.

| Label | Source | Resolution | Attempted | Measured GPU | total_ms | dispatch_ms | readback_ms | ms/MPixel | scaling ratio vs native | approx linear |
| --- | --- | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | --- |
| native_imported | native_imported | 256x144 | true | true | 0.4926 | 0.4209 | 0.0705 | 13.3626 | n/a | unknown |
| scaled_1080p | scaled_external_ready | 1920x1080 | true | true | 16.3600 | 11.7959 | 4.5612 | 7.8897 | 33.2117 | false |
| scaled_4k | scaled_external_ready | 3840x2160 | true | true | 70.7708 | 53.4698 | 17.2971 | 8.5324 | 143.6685 | false |

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

- Real imported captures still need the same scaling study on the target evaluator hardware.
