# GPU Execution Report

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

measured_gpu: `true`
measurement_kind: `measured_gpu`
kernel: `dsfb_host_minimum`

NO REAL EXTERNAL DATA PROVIDED

| Capture | measured_gpu | adapter | backend | resolution | total_ms | dispatch_ms | readback_ms | trust_delta_vs_cpu | alpha_delta_vs_cpu | intervention_delta_vs_cpu |
| --- | --- | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| capture_0 | true | NVIDIA GeForce RTX 4080 SUPER | Vulkan | 160x96 | 0.3605 | 0.3082 | 0.0505 | 0.000000 | 0.000000 | 0.000000 |

## What Is Proven

- The imported external buffers can execute through the same minimum host-realistic GPU kernel as the internal study.
- GPU-vs-CPU numerical deltas are recorded whenever a GPU adapter is available.

## What Is Not Proven

- This file does not prove production renderer integration or full engine-side GPU cost.
- If `measured_gpu` is `false`, the path is implemented but unmeasured in the current environment.

## Remaining Blockers

- Engine-exported captures on the target evaluation hardware still need GPU-side profiling.
