# External Scaling Report

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

Native imported capture: `ambush_5_mixed_frame_0047` at 1024x436.

| Label | Source | Resolution | Attempted | Measured GPU | total_ms | dispatch_ms | readback_ms | ms/MPixel | scaling ratio vs native | approx linear |
| --- | --- | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | --- |
| native_imported | native_imported | 1024x436 | true | true | 5.2039 | 3.5473 | 1.6545 | 11.6558 | n/a | unknown |
| scaled_1080p | scaled_external_ready | 1920x1080 | true | true | 17.7912 | 12.5839 | 5.2052 | 8.5798 | 3.4188 | false |
| scaled_4k | scaled_external_ready | 3840x2160 | true | false | n/a | n/a | n/a | n/a | n/a | unknown |
  - unavailable: GPU scaling attempt failed at runtime: wgpu error: Validation Error

Caused by:
    In Device::create_bind_group
      note: label = `dsfb-host-minimum-bind-group`
    Buffer binding 4 range 265420800 exceeds `max_*_buffer_binding_size` limit 134217728



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
