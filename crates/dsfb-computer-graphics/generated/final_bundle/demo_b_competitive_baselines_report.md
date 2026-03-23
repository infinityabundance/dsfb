# Demo B Competitive Baselines Report

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

This report compares imported trust against the full heuristic suite: gradient-magnitude / edge-guided, residual-guided, contrast-guided, variance-guided, combined heuristic, native trust, and hybrid trust + variance.

| Scenario | Taxonomy | Best heuristic baseline | Best heuristic ROI MAE | Imported trust ROI MAE | Hybrid ROI MAE | Interpretation |
| --- | --- | --- | ---: | ---: | ---: | --- |
| thin_reveal | aliasing_limited | Edge-guided | 0.03184 | 0.03184 | 0.17200 | Heuristic baseline remains competitive here. |
| fast_pan | mixed | Contrast-guided | 0.00537 | 0.00220 | 0.00381 | Imported trust beats the strongest heuristic baseline on fixed budget. |
| diagonal_reveal | aliasing_limited | Edge-guided | 0.00527 | 0.00770 | 0.01245 | Heuristic baseline remains competitive here. |
| reveal_band | mixed | Contrast-guided | 0.00929 | 0.00343 | 0.00533 | Imported trust beats the strongest heuristic baseline on fixed budget. |
| motion_bias_band | mixed | Contrast-guided | 0.01702 | 0.00974 | 0.01498 | Imported trust beats the strongest heuristic baseline on fixed budget. |
| layered_slats | mixed | Combined heuristic | 0.00881 | 0.00353 | 0.00763 | Imported trust beats the strongest heuristic baseline on fixed budget. |
| noisy_reprojection | variance_limited | Contrast-guided | 0.01992 | 0.01056 | 0.01499 | Imported trust beats the strongest heuristic baseline on fixed budget. |
| heuristic_friendly_pan | edge_trap | Combined heuristic | 0.00692 | 0.00252 | 0.00475 | Imported trust beats the strongest heuristic baseline on fixed budget. |
| contrast_pulse | variance_limited | Edge-guided | 0.00008 | 0.00008 | 0.00008 | Heuristic baseline remains competitive here. |
| stability_holdout | variance_limited | Contrast-guided | 0.00348 | 0.00672 | 0.00692 | Heuristic baseline remains competitive here. |

## What Is Not Proven

- This report does not claim the same ranking will hold on externally replayed renderer traces.

## Remaining Blockers

- External sample-allocation traces and real renderer variance are still needed before these competitive-baseline rankings become externally validated.
