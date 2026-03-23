# Resolution Scaling Report

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

| Tier | Scenario | Resolution | ROI pixels | ROI fraction | Host ROI MAE | Host vs fixed gain | Motion vs host gain | Memory MB |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| default_full_suite | thin_reveal | 160x96 | 1 | 0.00007 | 0.34793 | 2.49573 | 0.12613 | 0.47 |
| default_full_suite | reveal_band | 160x96 | 156 | 0.01016 | 0.17590 | 1.83719 | 0.05385 | 0.47 |
| default_full_suite | motion_bias_band | 160x96 | 714 | 0.04648 | 0.42329 | 2.52368 | 0.00386 | 0.47 |
| default_full_suite | contrast_pulse | 160x96 | 1872 | 0.12188 | 0.00000 | -0.00000 | 0.00000 | 0.47 |
| intermediate_selected_suite | reveal_band | 640x360 | 156 | 0.00068 | 0.20736 | 2.16330 | 0.06377 | 7.03 |
| intermediate_selected_suite | motion_bias_band | 640x360 | 12242 | 0.05313 | 0.48803 | 3.48957 | 0.00030 | 7.03 |
| intermediate_selected_suite | contrast_pulse | 640x360 | 1872 | 0.00812 | 0.00000 | -0.00000 | 0.00000 | 7.03 |
| high_resolution_proxy_selected_suite | reveal_band | 960x540 | 156 | 0.00030 | 0.20997 | 2.18996 | 0.06456 | 15.82 |
| high_resolution_proxy_selected_suite | motion_bias_band | 960x540 | 29210 | 0.05635 | 0.49093 | 3.51246 | 0.00013 | 15.82 |
| high_resolution_proxy_selected_suite | contrast_pulse | 960x540 | 1872 | 0.00361 | 0.00000 | -0.00000 | 0.00000 | 15.82 |

- The high-resolution tier is a selected-scenario scalable proxy rather than a full 1080p sweep. It is intended to demonstrate structural persistence beyond the toy default resolution without pretending to be a shipping-engine benchmark.
- The canonical thin_reveal point-ROI case is intentionally kept at the default resolution only. At higher resolutions its exact one-pixel disocclusion geometry becomes path-dependent and is not a stable scaling metric.
- Memory footprint numbers are analytical host-realistic buffer estimates from the crate cost model.

## What Is Not Proven

- This report is a structural scaling study, not a production-scene benchmark.

## Remaining Blockers

- A full 1080p or 4K full-suite run with real hardware timing remains future work.
