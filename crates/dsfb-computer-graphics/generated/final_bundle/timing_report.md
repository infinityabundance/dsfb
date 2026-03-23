# Timing Report

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

Measurement classification: `cpu_only_proxy`.

Actual GPU timing measured: `false`.

- No actual GPU timing was measured in this environment. These timings are CPU-side proxy measurements of the same per-pixel supervisory structure and are paired with analytical op and memory estimates.
- The highest-resolution entry is a selected-scenario host-realistic proxy, not a full-suite production benchmark.

| Label | Mode | Scenario | Resolution | Build | Total ms | ms / frame | Ops / px | Traffic MB |
| --- | --- | --- | --- | --- | ---: | ---: | ---: | ---: |
| minimum_host_path_default_res | minimal | thin_reveal | 160x96 | release | 139.112 | 1.364 | 20 | 53.79 |
| motion_augmented_region_mid_res | host_realistic | motion_bias_band | 640x360 | release | 1958.343 | 28.799 | 60 | 1852.73 |
| full_debug_region_mid_res | full_research_debug | reveal_band | 640x360 | release | 2086.214 | 30.680 | 66 | 2390.62 |
| minimum_host_path_high_res_proxy | host_realistic | reveal_band | 1920x1080 | release | 3411.731 | 200.690 | 60 | 4168.65 |

## Per-Stage Breakdown

### minimum_host_path_default_res

| Stage | Total ms | ms / frame | ns / pixel |
| --- | ---: | ---: | ---: |
| reproject | 37.626 | 0.369 | 24.016 |
| supervise | 97.268 | 0.954 | 62.084 |
| resolve | 4.084 | 0.040 | 2.607 |

Likely optimization levers:
- Fuse alpha modulation into the temporal resolve.
- Compute trust/intervention at half resolution if only gating is needed.

### motion_augmented_region_mid_res

| Stage | Total ms | ms / frame | ns / pixel |
| --- | ---: | ---: | ---: |
| reproject | 450.458 | 6.624 | 28.752 |
| supervise | 1462.585 | 21.509 | 93.353 |
| resolve | 43.070 | 0.633 | 2.749 |

Likely optimization levers:
- Fuse reprojection fetches across color, depth, and normal buffers.
- Evaluate trust at half resolution or per tile, then upsample alpha.
- Keep motion disagreement optional; the minimum path no longer pays for it when scenario evidence is weak.

### full_debug_region_mid_res

| Stage | Total ms | ms / frame | ns / pixel |
| --- | ---: | ---: | ---: |
| reproject | 392.830 | 5.777 | 25.073 |
| supervise | 1451.993 | 21.353 | 92.677 |
| resolve | 42.685 | 0.628 | 2.725 |

Likely optimization levers:
- Drop synthetic visibility and debug exports outside analysis mode.
- Compress trust/alpha/intervention into narrower formats once calibration work stabilizes.

### minimum_host_path_high_res_proxy

| Stage | Total ms | ms / frame | ns / pixel |
| --- | ---: | ---: | ---: |
| reproject | 906.907 | 53.347 | 25.727 |
| supervise | 2427.871 | 142.816 | 68.873 |
| resolve | 73.831 | 4.343 | 2.094 |

Likely optimization levers:
- Fuse reprojection fetches across color, depth, and normal buffers.
- Evaluate trust at half resolution or per tile, then upsample alpha.
- Keep motion disagreement optional; the minimum path no longer pays for it when scenario evidence is weak.

## What Is Not Proven

- This report does not contain measured GPU milliseconds.
- It does not justify any production deployment performance claim.

## Remaining Blockers

- Real GPU execution and memory-system measurements remain outstanding.
