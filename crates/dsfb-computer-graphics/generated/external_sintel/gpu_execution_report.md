# GPU Execution Report

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

measured_gpu: `true`
measurement_kind: `measured_gpu`
kernel: `dsfb_host_minimum`

| Capture | measured_gpu | adapter | backend | resolution | total_ms | dispatch_ms | readback_ms | trust_delta_vs_cpu | alpha_delta_vs_cpu | intervention_delta_vs_cpu |
| --- | --- | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| ambush_5_mixed_frame_0047 | true | NVIDIA GeForce RTX 4080 SUPER | Vulkan | 1024x436 | 4.7045 | 3.4956 | 1.2071 | 0.000000 | 0.000000 | 0.000000 |
| ambush_5_point_frame_0047 | true | NVIDIA GeForce RTX 4080 SUPER | Vulkan | 1024x436 | 4.2031 | 3.5167 | 0.6838 | 0.000000 | 0.000000 | 0.000000 |
| ambush_5_region_frame_0047 | true | NVIDIA GeForce RTX 4080 SUPER | Vulkan | 1024x436 | 4.1064 | 3.4947 | 0.6099 | 0.000000 | 0.000000 | 0.000000 |
| market_6_mixed_frame_0008 | true | NVIDIA GeForce RTX 4080 SUPER | Vulkan | 1024x436 | 4.0621 | 3.5045 | 0.5555 | 0.000000 | 0.000000 | 0.000000 |
| market_6_region_frame_0008 | true | NVIDIA GeForce RTX 4080 SUPER | Vulkan | 1024x436 | 4.1113 | 3.4776 | 0.6310 | 0.000000 | 0.000000 | 0.000000 |

## What Is Proven

- The imported external buffers can execute through the same minimum host-realistic GPU kernel as the internal study.
- GPU-vs-CPU numerical deltas are recorded whenever a GPU adapter is available.

## What Is Not Proven

- This file does not prove production renderer integration or full engine-side GPU cost.
- If `measured_gpu` is `false`, the path is implemented but unmeasured in the current environment.

## Remaining Blockers

- Engine-exported captures on the target evaluation hardware still need GPU-side profiling.
