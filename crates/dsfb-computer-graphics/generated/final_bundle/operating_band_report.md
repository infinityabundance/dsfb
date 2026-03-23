# Operating Band Report

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

This report translates parameter sweeps into evaluator-facing operating bands: what is robust, what is moderately sensitive, and what is fragile.

| Parameter | Robust values | Moderately sensitive values | Fragile values | First tuning priority |
| --- | --- | --- | --- | --- |
| alpha_max | 0.960, 0.990 | none | 0.840 | second |
| alpha_min | 0.080, 0.120 | none | 0.040 | second |
| depth_weight | 1.000, 1.250, 1.500 | 0.750 | 0.500 | first |
| grammar_weight | 1.000, 1.250, 1.500 | 0.750 | 0.500 | first |
| motion_weight | 0.500, 0.750, 1.000, 1.250, 1.500 | none | none | optional path only |
| residual_threshold_scale | 0.500, 0.750, 1.000 | 1.250 | 1.500 | first |
| thin_weight | 0.750, 1.000, 1.250, 1.500 | 0.500 | none | first |

## What Is Proven

- The current weights are no longer opaque magic constants; they are centralized and classified into safe, narrower, and fragile corridors.

## What Is Not Proven

- These operating bands are still derived from synthetic in-crate sweeps rather than externally validated calibration.

## Remaining Blockers

- External replay and engine-side tuning are still required before these operating bands can be treated as deployment guidance.
