# Timing Report

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

Measurement classification: `cpu_only_proxy`.

Actual GPU timing measured: `false`.

- No actual GPU timing was measured in this environment. These timings are CPU-side proxy measurements of the same per-pixel supervisory structure and are paired with analytical op and memory estimates.
- The highest-resolution entry is a selected-scenario host-realistic proxy, not a full-suite production benchmark.

| Label | Mode | Scenario | Resolution | Build | Total ms | ms / frame | Ops / px | Traffic MB |
| --- | --- | --- | --- | --- | ---: | ---: | ---: | ---: |
| minimum_host_path_default_res | minimal | thin_reveal | 160x96 | release | 134.789 | 1.321 | 20 | 53.79 |
| motion_augmented_region_mid_res | host_realistic | motion_bias_band | 640x360 | release | 1864.221 | 27.415 | 60 | 1852.73 |
| full_debug_region_mid_res | full_research_debug | reveal_band | 640x360 | release | 1997.188 | 29.370 | 66 | 2390.62 |
| minimum_host_path_high_res_proxy | host_realistic | reveal_band | 1920x1080 | release | 3234.668 | 190.275 | 60 | 4168.65 |

## Per-Stage Breakdown

### minimum_host_path_default_res

| Stage | Total ms | ms / frame | ns / pixel |
| --- | ---: | ---: | ---: |
| reproject | 36.704 | 0.360 | 23.427 |
| supervise | 95.752 | 0.939 | 61.116 |
| resolve | 2.264 | 0.022 | 1.445 |

Likely optimization levers:
- Fuse alpha modulation into the temporal resolve.
- Compute trust/intervention at half resolution if only gating is needed.

### motion_augmented_region_mid_res

| Stage | Total ms | ms / frame | ns / pixel |
| --- | ---: | ---: | ---: |
| reproject | 419.551 | 6.170 | 26.779 |
| supervise | 1406.400 | 20.682 | 89.767 |
| resolve | 36.830 | 0.542 | 2.351 |

Likely optimization levers:
- Fuse reprojection fetches across color, depth, and normal buffers.
- Evaluate trust at half resolution or per tile, then upsample alpha.
- Keep motion disagreement optional; the minimum path no longer pays for it when scenario evidence is weak.

### full_debug_region_mid_res

| Stage | Total ms | ms / frame | ns / pixel |
| --- | ---: | ---: | ---: |
| reproject | 366.639 | 5.392 | 23.402 |
| supervise | 1388.813 | 20.424 | 88.645 |
| resolve | 38.477 | 0.566 | 2.456 |

Likely optimization levers:
- Drop synthetic visibility and debug exports outside analysis mode.
- Compress trust/alpha/intervention into narrower formats once calibration work stabilizes.

### minimum_host_path_high_res_proxy

| Stage | Total ms | ms / frame | ns / pixel |
| --- | ---: | ---: | ---: |
| reproject | 844.806 | 49.694 | 23.965 |
| supervise | 2320.656 | 136.509 | 65.832 |
| resolve | 66.009 | 3.883 | 1.873 |

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
