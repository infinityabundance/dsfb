# Competitive Baseline Analysis

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

Recommended framing: **targeted supervisory overlay / instability-focused specialist**.

| Scenario | Competitive baseline case | Host vs strong ROI gain | Non-ROI penalty ratio vs strong | Interpretation |
| --- | --- | ---: | ---: | --- |
| thin_reveal | false | 0.23014 | 1.36596 | DSFB wins in the targeted instability region. |
| fast_pan | true | -0.04395 | 1.48707 | Strong heuristic remains competitive or better here. |
| diagonal_reveal | false | 0.41377 | 0.92272 | DSFB wins in the targeted instability region. |
| reveal_band | false | 0.06868 | 0.81020 | DSFB wins in the targeted instability region. |
| motion_bias_band | false | 0.33513 | 0.81862 | DSFB wins in the targeted instability region. |
| layered_slats | false | 0.11961 | 0.86708 | DSFB wins in the targeted instability region. |
| noisy_reprojection | false | 0.74017 | 1.14974 | DSFB wins in the targeted instability region. |
| heuristic_friendly_pan | true | 0.11158 | 0.73889 | DSFB wins in the targeted instability region. |
| contrast_pulse | false | -0.00000 | 1.00000 | Tie or effectively neutral. |
| stability_holdout | false | -0.00000 | 1.00000 | Tie or effectively neutral. |

## What Is Not Proven

- This analysis does not support universal-win language.
- External validation is still required to confirm these competitive-baseline relationships on imported engine captures.

## Remaining Blockers

- Competitive-baseline results still need real-engine confirmation on imported captures.
