# GPU Execution Report

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

Measurement classification: `actual_gpu_timing_measured`.

Actual GPU timing measured: `true`.

Kernel: `dsfb_host_minimum` in `wgsl`.

| Label | Scenario | Resolution | Tier | Measured | Adapter | Total ms | Dispatch ms | Readback ms | Trust delta vs CPU |
| --- | --- | --- | --- | --- | --- | ---: | ---: | ---: | ---: |
| gpu_host_minimum_reveal_band | reveal_band | 160x96 | native | true | NVIDIA GeForce RTX 4080 SUPER | 0.315 | 0.272 | 0.042 | 0.000000 |
| gpu_host_minimum_motion_bias_band | motion_bias_band | 160x96 | native | true | NVIDIA GeForce RTX 4080 SUPER | 0.299 | 0.262 | 0.036 | 0.000000 |
| gpu_4k_synthetic_probe | synthetic_4k | 3840x2160 | 4k_probe | true | NVIDIA GeForce RTX 4080 SUPER | 76.274 | 52.856 | 23.415 | n/a |

## GPU Path Status

- This path is intended to remove the 'CPU-only timing proxy' blocker by providing a real GPU-executable kernel and an honest measured-vs-unmeasured disclosure.
- The current kernel covers the minimum host-realistic supervisory path. Motion disagreement remains an optional extension and is not part of the minimum kernel.

## How To Run On A GPU Host

```bash
cargo run --release -- run-gpu-path --output generated/gpu_path
```

## What Is Not Proven

- This report does not imply measured GPU performance when `Actual GPU timing measured` is `false`.
- It does not replace real engine-side GPU profiling or cache/bandwidth measurement.

## Remaining Blockers

- The kernel is measured, but broader engine-integrated GPU profiling still remains.
- Real engine captures and imported external buffers still need GPU-side evaluation.
