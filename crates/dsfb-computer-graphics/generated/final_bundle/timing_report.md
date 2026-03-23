# Timing Report

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

Measurement classification: `cpu_only_proxy`.

Actual GPU timing measured: `false`.

- No actual GPU timing was measured in this environment. These timings are CPU-side proxy measurements of the same per-pixel supervisory structure and are paired with analytical op and memory estimates.
- The highest-resolution entry is a selected-scenario host-realistic proxy, not a full-suite production benchmark.

| Label | Mode | Scenario | Resolution | Build | Total ms | ms / frame | Ops / px | Traffic MB |
| --- | --- | --- | --- | --- | ---: | ---: | ---: | ---: |
| minimum_host_path_default_res | minimal | thin_reveal | 160x96 | release | 133.609 | 1.310 | 20 | 53.79 |
| motion_augmented_region_mid_res | host_realistic | motion_bias_band | 640x360 | release | 1862.457 | 27.389 | 60 | 1852.73 |
| full_debug_region_mid_res | full_research_debug | reveal_band | 640x360 | release | 1998.859 | 29.395 | 66 | 2390.62 |
| minimum_host_path_high_res_proxy | host_realistic | reveal_band | 1920x1080 | release | 3199.774 | 188.222 | 60 | 4168.65 |

## Per-Stage Breakdown

### minimum_host_path_default_res

| Stage | Total ms | ms / frame | ns / pixel |
| --- | ---: | ---: | ---: |
| reproject | 36.812 | 0.361 | 23.496 |
| supervise | 95.205 | 0.933 | 60.767 |
| resolve | 1.503 | 0.015 | 0.959 |

Likely optimization levers:
- Fuse alpha modulation into the temporal resolve.
- Compute trust/intervention at half resolution if only gating is needed.

### motion_augmented_region_mid_res

| Stage | Total ms | ms / frame | ns / pixel |
| --- | ---: | ---: | ---: |
| reproject | 420.985 | 6.191 | 26.870 |
| supervise | 1409.102 | 20.722 | 89.940 |
| resolve | 31.030 | 0.456 | 1.981 |

Likely optimization levers:
- Fuse reprojection fetches across color, depth, and normal buffers.
- Evaluate trust at half resolution or per tile, then upsample alpha.
- Keep motion disagreement optional; the minimum path no longer pays for it when scenario evidence is weak.

### full_debug_region_mid_res

| Stage | Total ms | ms / frame | ns / pixel |
| --- | ---: | ---: | ---: |
| reproject | 369.840 | 5.439 | 23.606 |
| supervise | 1405.738 | 20.673 | 89.725 |
| resolve | 31.504 | 0.463 | 2.011 |

Likely optimization levers:
- Drop synthetic visibility and debug exports outside analysis mode.
- Compress trust/alpha/intervention into narrower formats once calibration work stabilizes.

### minimum_host_path_high_res_proxy

| Stage | Total ms | ms / frame | ns / pixel |
| --- | ---: | ---: | ---: |
| reproject | 857.550 | 50.444 | 24.327 |
| supervise | 2277.593 | 133.976 | 64.610 |
| resolve | 61.295 | 3.606 | 1.739 |

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
