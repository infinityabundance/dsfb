# Demo B Aliasing vs Variance Report

| Scenario | Demo B taxonomy | Imported trust ROI MAE | Uniform ROI MAE | Combined heuristic ROI MAE |
| --- | --- | ---: | ---: | ---: |
| thin_reveal | aliasing_limited | 0.03184 | 0.17226 | 0.04977 |
| fast_pan | mixed | 0.00220 | 0.00804 | 0.00603 |
| diagonal_reveal | aliasing_limited | 0.00770 | 0.01607 | 0.01245 |
| reveal_band | mixed | 0.00343 | 0.01459 | 0.00972 |
| motion_bias_band | mixed | 0.00974 | 0.03229 | 0.01716 |
| layered_slats | mixed | 0.00353 | 0.01985 | 0.00881 |
| noisy_reprojection | variance_limited | 0.01056 | 0.03290 | 0.02067 |
| heuristic_friendly_pan | edge_trap | 0.00252 | 0.01030 | 0.00692 |
| contrast_pulse | variance_limited | 0.00008 | 0.00008 | 0.00008 |
| stability_holdout | variance_limited | 0.00672 | 0.00672 | 0.00363 |

## What Is Not Proven

- This report does not claim the same ordering will hold under real renderer variance or path-tracing noise.
- External validation is still required before treating aliasing-vs-variance separation as an engine-level conclusion.

## Remaining Blockers

- Real renderer noise and in-engine sample allocation remain future work.
- No external renderer handoff exists yet for per-pixel sample-allocation traces.
