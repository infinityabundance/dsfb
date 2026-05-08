# Per-axis (tier) discrimination audit

Discrimination ratio = mean_fault_rate / (mean_healthy_rate + 0.01)
aggregated across all detectors in each axis, across all 12 fixtures.

Higher = axis fires more selectively on fault windows.
A ratio near 1.0 indicates the axis fires equally on healthy/fault.
A ratio near 0.0 indicates the axis is dormant on the current surface.

Source: Phase ζ.6 audit harness (`src/audit/axis_discrimination.rs`).

| Tier | Bit | Detectors | Fixtures | Mean healthy | Mean fault | Discrimination |
|:----:|----:|----------:|---------:|-------------:|-----------:|---------------:|
| F | 0x00000020 | 48 | 12 | 0.2889 | 0.0417 | 0.1394 |
| B | 0x00000002 | 12 | 12 | 0.3806 | 0.0000 | 0.0000 |
| D | 0x00000008 | 19 | 12 | 0.1094 | 0.0000 | 0.0000 |
| E | 0x00000010 | 12 | 12 | 0.0000 | 0.0000 | 0.0000 |
| M | 0x00002000 | 12 | 12 | 0.0008 | 0.0000 | 0.0000 |
| S | 0x00080000 | 12 | 12 | 0.0010 | 0.0000 | 0.0000 |
