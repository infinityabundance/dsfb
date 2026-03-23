# Non-ROI Penalty Report

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

This report quantifies non-ROI penalty so evaluator-facing claims do not hide off-target cost.

| Scenario | Host non-ROI MAE penalty vs fixed | Host non-ROI MAE penalty vs strong | Penalty ratio vs strong | Note |
| --- | ---: | ---: | ---: | --- |
| thin_reveal | -0.01139 | 0.00028 | 1.36596 | Host-realistic DSFB remains competitive without privileged visibility hints on this scenario. |
| fast_pan | -0.01923 | 0.00058 | 1.48707 | Strong heuristic remains better on this scenario; the report surfaces that rather than hiding it. |
| diagonal_reveal | -0.00815 | -0.00005 | 0.92272 | Host-realistic DSFB remains competitive without privileged visibility hints on this scenario. |
| reveal_band | -0.02841 | -0.00077 | 0.81020 | Host-realistic DSFB remains competitive without privileged visibility hints on this scenario. |
| motion_bias_band | -0.03690 | -0.00174 | 0.81862 | Host-realistic DSFB remains competitive without privileged visibility hints on this scenario. |
| layered_slats | -0.03242 | -0.00058 | 0.86708 | Host-realistic DSFB remains competitive without privileged visibility hints on this scenario. |
| noisy_reprojection | -0.03566 | 0.00146 | 1.14974 | Host-realistic DSFB remains competitive without privileged visibility hints on this scenario. |
| heuristic_friendly_pan | -0.01732 | -0.00073 | 0.73889 | Host-realistic DSFB remains competitive without privileged visibility hints on this scenario. |
| contrast_pulse | 0.00000 | 0.00000 | 1.00000 | This is the honesty scenario: aggressive trust collapse is not expected to help, so false-positive response and non-ROI stability are the main evaluation criteria. |
| stability_holdout | 0.00000 | 0.00000 | 1.00000 | This is the honesty scenario: aggressive trust collapse is not expected to help, so false-positive response and non-ROI stability are the main evaluation criteria. |

## What Is Not Proven

- This report does not claim DSFB improves global full-frame quality in every case.

## Remaining Blockers

- Non-ROI tradeoffs still need validation on imported external captures and measured GPU runs.
