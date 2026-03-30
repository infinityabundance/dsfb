# GPU Execution Report

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

measured_gpu: `true`
measurement_kind: `measured_gpu`
kernel: `dsfb_host_minimum`

| Capture | measured_gpu | adapter | backend | resolution | total_ms | dispatch_ms | readback_ms | trust_delta_vs_cpu | alpha_delta_vs_cpu | intervention_delta_vs_cpu |
| --- | --- | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| frame_0001 | true | llvmpipe (LLVM 22.1.1, 256 bits) | Gl | 256x144 | 19.4802 | 19.3927 | 0.0869 | 0.000000 | 0.000000 | 0.000000 |
| frame_0002 | true | llvmpipe (LLVM 22.1.1, 256 bits) | Gl | 256x144 | 19.5371 | 19.4499 | 0.0867 | 0.000000 | 0.000000 | 0.000000 |
| frame_0003 | true | llvmpipe (LLVM 22.1.1, 256 bits) | Gl | 256x144 | 18.9144 | 18.8265 | 0.0871 | 0.000000 | 0.000000 | 0.000000 |
| frame_0004 | true | llvmpipe (LLVM 22.1.1, 256 bits) | Gl | 256x144 | 18.3180 | 18.2410 | 0.0767 | 0.000000 | 0.000000 | 0.000000 |
| frame_0005 | true | llvmpipe (LLVM 22.1.1, 256 bits) | Gl | 256x144 | 18.4018 | 18.3200 | 0.0814 | 0.000000 | 0.000000 | 0.000000 |

## What Is Proven

- The imported external buffers can execute through the same minimum host-realistic GPU kernel as the internal study.
- GPU-vs-CPU numerical deltas are recorded whenever a GPU adapter is available.

## What Is Not Proven

- This file does not prove production renderer integration or full engine-side GPU cost.
- If `measured_gpu` is `false`, the path is implemented but unmeasured in the current environment.

## Remaining Blockers

- Engine-exported captures on the target evaluation hardware still need GPU-side profiling.
