# Timing Report

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

Measurement classification: `cpu_only_proxy`.

Actual GPU timing measured: `false`.

- No actual GPU timing was measured in this environment. These timings are CPU-side proxy measurements of the same per-pixel supervisory structure and are paired with analytical op and memory estimates.
- The highest-resolution entry is a selected-scenario host-realistic proxy, not a full-suite production benchmark.

| Label | Mode | Scenario | Resolution | Build | Total ms | ms / frame | Ops / px | Traffic MB |
| --- | --- | --- | --- | --- | ---: | ---: | ---: | ---: |
| minimum_host_path_default_res | minimal | thin_reveal | 160x96 | release | 140.409 | 1.377 | 20 | 53.79 |
| motion_augmented_region_mid_res | host_realistic | motion_bias_band | 640x360 | release | 1929.906 | 28.381 | 60 | 1852.73 |
| full_debug_region_mid_res | full_research_debug | reveal_band | 640x360 | release | 2081.778 | 30.614 | 66 | 2390.62 |
| minimum_host_path_high_res_proxy | host_realistic | reveal_band | 1920x1080 | release | 3251.485 | 191.264 | 60 | 4168.65 |

## Per-Stage Breakdown

### minimum_host_path_default_res

| Stage | Total ms | ms / frame | ns / pixel |
| --- | ---: | ---: | ---: |
| reproject | 38.544 | 0.378 | 24.602 |
| supervise | 99.401 | 0.975 | 63.446 |
| resolve | 2.391 | 0.023 | 1.526 |

Likely optimization levers:
- Fuse alpha modulation into the temporal resolve.
- Compute trust/intervention at half resolution if only gating is needed.

### motion_augmented_region_mid_res

| Stage | Total ms | ms / frame | ns / pixel |
| --- | ---: | ---: | ---: |
| reproject | 433.501 | 6.375 | 27.669 |
| supervise | 1454.746 | 21.393 | 92.853 |
| resolve | 40.285 | 0.592 | 2.571 |

Likely optimization levers:
- Fuse reprojection fetches across color, depth, and normal buffers.
- Evaluate trust at half resolution or per tile, then upsample alpha.
- Keep motion disagreement optional; the minimum path no longer pays for it when scenario evidence is weak.

### full_debug_region_mid_res

| Stage | Total ms | ms / frame | ns / pixel |
| --- | ---: | ---: | ---: |
| reproject | 383.413 | 5.638 | 24.472 |
| supervise | 1449.797 | 21.321 | 92.537 |
| resolve | 39.995 | 0.588 | 2.553 |

Likely optimization levers:
- Drop synthetic visibility and debug exports outside analysis mode.
- Compress trust/alpha/intervention into narrower formats once calibration work stabilizes.

### minimum_host_path_high_res_proxy

| Stage | Total ms | ms / frame | ns / pixel |
| --- | ---: | ---: | ---: |
| reproject | 858.319 | 50.489 | 24.349 |
| supervise | 2320.287 | 136.487 | 65.822 |
| resolve | 69.937 | 4.114 | 1.984 |

Likely optimization levers:
- Fuse reprojection fetches across color, depth, and normal buffers.
- Evaluate trust at half resolution or per tile, then upsample alpha.
- Keep motion disagreement optional; the minimum path no longer pays for it when scenario evidence is weak.

## What Is Not Proven

- This report does not contain measured GPU milliseconds.
- It does not justify any production deployment performance claim.
- External validation is still required on real engine-exported buffers and target GPU hardware.

## Remaining Blockers

- Real GPU execution and memory-system measurements remain outstanding.
- External handoff is available, but externally validated timing data is still absent.
